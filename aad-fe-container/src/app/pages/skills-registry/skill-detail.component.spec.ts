import { ComponentFixture, TestBed } from '@angular/core/testing';
import { SkillDetailComponent } from './skill-detail.component';
import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';
import { ApiService } from '../../services/api.service';
import { NO_ERRORS_SCHEMA } from '@angular/core';

describe('SkillDetailComponent', () => {
  let component: SkillDetailComponent;
  let fixture: ComponentFixture<SkillDetailComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [SkillDetailComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([]),
        {
          provide: ActivatedRoute,
          useValue: {
            params: of({}),
            queryParams: of({}),
            snapshot: { paramMap: { get: () => null }, routeConfig: { path: '' } },
            paramMap: of({ get: () => null }),
          },
        },
        {
          provide: ApiService,
          useValue: {
            getSkills: () => of([]),
            getTools: () => of([]),
            getTraits: () => of({ ids: [] }),
            getTrait: () => of(null),
          },
        },
      ],
      schemas: [NO_ERRORS_SCHEMA],
    }).compileComponents();

    fixture = TestBed.createComponent(SkillDetailComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create the skills registry component', () => {
    expect(component).toBeTruthy();
  });


});
