import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SkillsRegistryLayoutComponent } from './skills-registry-layout.component';
import { HttpClientTestingModule } from '@angular/common/http/testing';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';

describe('SkillsRegistryLayoutComponent', () => {
  let component: SkillsRegistryLayoutComponent;
  let fixture: ComponentFixture<SkillsRegistryLayoutComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [SkillsRegistryLayoutComponent, HttpClientTestingModule],
      providers: [{ provide: ActivatedRoute, useValue: { params: of({}) } }]
    })
    .compileComponents();

    fixture = TestBed.createComponent(SkillsRegistryLayoutComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
