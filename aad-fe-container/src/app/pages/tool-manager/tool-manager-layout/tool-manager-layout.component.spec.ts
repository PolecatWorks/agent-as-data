import { HttpClientTestingModule } from '@angular/common/http/testing';
import { RouterTestingModule } from '@angular/router/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ToolManagerLayoutComponent } from './tool-manager-layout.component';

describe('ToolManagerLayoutComponent', () => {
  let component: ToolManagerLayoutComponent;
  let fixture: ComponentFixture<ToolManagerLayoutComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HttpClientTestingModule, RouterTestingModule, ToolManagerLayoutComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ToolManagerLayoutComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
