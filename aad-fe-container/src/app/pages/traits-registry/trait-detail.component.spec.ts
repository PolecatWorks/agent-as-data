import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HttpClientTestingModule } from '@angular/common/http/testing';
import { RouterTestingModule } from '@angular/router/testing';
import { NoopAnimationsModule } from '@angular/platform-browser/animations';
import { TraitDetailComponent } from './trait-detail.component';
import { ApiService } from '../../services/api.service';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';

describe('TraitDetailComponent', () => {
  let component: TraitDetailComponent;
  let fixture: ComponentFixture<TraitDetailComponent>;
  let apiServiceSpy: jasmine.SpyObj<ApiService>;

  beforeEach(async () => {
    apiServiceSpy = jasmine.createSpyObj('ApiService', [
      'getTraits',
      'getTrait',
      'createTrait',
      'updateTrait',
      'deleteTrait',
    ]);
    apiServiceSpy.getTraits.and.returnValue(
      of({ ids: [], pagination: { page: 1, size: 10 } }),
    );

    await TestBed.configureTestingModule({
      imports: [
        TraitDetailComponent,
        HttpClientTestingModule,
        RouterTestingModule,
        NoopAnimationsModule,
      ],
      providers: [
        { provide: ApiService, useValue: apiServiceSpy },
        { 
          provide: ActivatedRoute, 
          useValue: { 
            paramMap: of({ get: () => null }),
            queryParams: of({}),
            snapshot: { routeConfig: { path: '' }, queryParams: {} }
          }
        }
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(TraitDetailComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create the component', () => {
    expect(component).toBeTruthy();
  });
});
